<?php

namespace App\Services;

use Illuminate\Support\Facades\DB;

class TelemetryService
{
    public function getLatestTelemetry(int $limit = 10): array
    {
        return DB::select("SELECT recorded_at, voltage, temp, source_file FROM telemetry_legacy ORDER BY recorded_at DESC LIMIT ?", [$limit]);
    }
}
