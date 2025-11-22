program LegacyCSV;
{$mode objfpc}{$H+}

uses
  Classes, SysUtils, DateUtils, Process;

// ← ВСПОМОГАТЕЛЬНАЯ ФУНКЦИЯ ДОЛЖНА БЫТЬ ЗДЕСЬ (до var и до begin)
function Env(const Name, Default: string): string;
var
  s: string;
begin
  s := GetEnvironmentVariable(Name);
  if s = '' then
    Result := Default
  else
    Result := s;
end;

procedure GenerateAndCopy;
var
  OutDir, FileName, FullPath: string;
  f: TextFile;
  Host, Port, UserName, Password, DBName: string;
  P: TProcess;
begin
  // Путь для CSV
  OutDir := Env('CSV_OUT_DIR', '/data/csv');
  FileName := 'telemetry_' + FormatDateTime('yyyymmdd_hhnnss', Now) + '.csv';
  FullPath := IncludeTrailingPathDelimiter(OutDir) + FileName;

  // Создаём CSV-файл
  AssignFile(f, FullPath);
  Rewrite(f);
  try
    WriteLn(f, 'recorded_at,voltage,temp,source_file');
    WriteLn(f,
      FormatDateTime('yyyy"-"mm"-"dd hh":"nn":"ss', Now) + ',' +
      FormatFloat('0.00', 3.2 + Random * (12.6 - 3.2)) + ',' +
      FormatFloat('0.00', -50 + Random * 130) + ',' +
      FileName);
  finally
    CloseFile(f);
  end;

  // Параметры PostgreSQL
  Host     := Env('PGHOST',      'db');
  Port     := Env('PGPORT',      '5432');
  UserName := Env('PGUSER',      'monouser');
  Password := Env('PGPASSWORD',  'monopass');
  DBName   := Env('PGDATABASE',  'monolith');

  // Запускаем psql \copy через TProcess
  P := TProcess.Create(nil);
  try
    P.Executable := 'psql';

    // ВАЖНО: передаём ВСЁ одной строкой через -d (connection string без пароля)
    P.Parameters.Add('-d');
    P.Parameters.Add(Format('postgresql://%s@%s:%s/%s',
      [UserName, Host, Port, DBName]));

    // Команда \copy
    P.Parameters.Add('-c');
    P.Parameters.Add(Format(
    '\copy telemetry_legacy (recorded_at, voltage, temp, source_file) FROM ''%s'' WITH (FORMAT csv, HEADER true)',
    [FullPath]));

    // Пароль — только через окружение (это безопасно и работает всегда)
    P.Environment.Add('PGPASSWORD=' + Password);

    P.Options := [poWaitOnExit];
    WriteLn('[pascal] Importing ', FileName, ' into PostgreSQL...');
    P.Execute;
  finally
    P.Free;
  end;
end;

// ──────────────────────────────────────────────────────────────
// Основная программа начинается здесь
var
  PeriodSec: Integer;
begin
  Randomize;
  PeriodSec := StrToIntDef(Env('GEN_PERIOD_SEC', '300'), 300);

  WriteLn('[pascal] LegacyCSV generator started (period = ', PeriodSec, ' sec)');

  while True do
  begin
    GenerateAndCopy;
    WriteLn('[pascal] ', FormatDateTime('hh:nn:ss', Now), ' — done');
    Sleep(PeriodSec * 1000);
  end;
end.