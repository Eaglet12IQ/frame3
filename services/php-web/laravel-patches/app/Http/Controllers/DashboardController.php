<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\DashboardService;
use App\Services\TelemetryService;

class DashboardController extends Controller
{
    protected $dashboardService;
    protected $telemetryService;

    public function __construct(DashboardService $dashboardService, TelemetryService $telemetryService)
    {
        $this->dashboardService = $dashboardService;
        $this->telemetryService = $telemetryService;
    }

    public function index(Request $request)
    {
        $dto = $this->dashboardService->getDashboardData();
        $data = $dto->toArray();

        $telemetry = $this->telemetryService->getLatestTelemetryPaginated(10);

        return view('dashboard', [
            'cms_block' => $data['cms_block'],
            'telemetry' => $telemetry,
        ]);
    }

    public function data()
    {
        $dto = $this->dashboardService->getDashboardData();
        return response()->json($dto->toArray());
    }

    public function downloadTelemetryCsv(Request $request)
    {
        $validated = $request->validate([
            'source_file' => 'required|string|max:255',
        ]);

        $sourceFile = $validated['source_file'];

        // Sanitize source_file to prevent path traversal
        $sourceFile = str_replace(['/', '\\', '..'], '', $sourceFile);

        $telemetry = $this->telemetryService->getTelemetryBySourceFile($sourceFile);

        if ($telemetry->isEmpty()) {
            abort(404, 'No telemetry data found for the specified source file');
        }

        $csvData = "Время записи,Напряжение (V),Температура (°C),Валидность,Источник\n";
        foreach ($telemetry as $record) {
            $csvData .= \Carbon\Carbon::parse($record['recorded_at'])->format('d.m.Y H:i:s') . ',';
            $csvData .= number_format($record['voltage'], 2) . ',';
            $csvData .= number_format($record['temp'], 2) . ',';
            $csvData .= ($record['is_valid'] ? 'Да' : 'Нет') . ',';
            $csvData .= $record['source_file'] . "\n";
        }

        $filename = 'telemetry_' . str_replace(['/', '\\', ' '], '_', $sourceFile) . '.csv';

        return response($csvData)
            ->header('Content-Type', 'text/csv')
            ->header('Content-Disposition', 'attachment; filename="' . $filename . '"');
    }
}
