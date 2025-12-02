<?php

namespace App\Services;

use Illuminate\Support\Facades\DB;

class DashboardService
{
    protected $issService;
    protected $telemetryService;

    public function __construct(IssService $issService, TelemetryService $telemetryService)
    {
        $this->issService = $issService;
        $this->telemetryService = $telemetryService;
    }

    public function getDashboardData(): array
    {
        $iss = $this->issService->getLast();
        $telemetry = $this->telemetryService->getLatestTelemetry(10);

        return [
            'iss' => $iss,
            'trend' => [], // Front-end will fetch /api/iss/trend via nginx proxy
            'jw_gallery' => [], // Not needed server-side
            'jw_observation_raw' => [],
            'jw_observation_summary' => [],
            'jw_observation_images' => [],
            'jw_observation_files' => [],
            'metrics' => [
                'iss_speed' => $iss['payload']['velocity'] ?? null,
                'iss_alt' => $iss['payload']['altitude'] ?? null,
                'neo_total' => 0,
            ],
            'telemetry' => $telemetry,
        ];
    }
}
