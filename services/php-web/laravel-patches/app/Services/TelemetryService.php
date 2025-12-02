<?php

namespace App\Services;

use App\Repositories\TelemetryRepository;

class TelemetryService
{
    protected $telemetryRepository;

    public function __construct(TelemetryRepository $telemetryRepository)
    {
        $this->telemetryRepository = $telemetryRepository;
    }

    public function getLatestTelemetry(int $limit = 10)
    {
        return $this->telemetryRepository->getLatestTelemetry($limit);
    }

    public function getTelemetryBySourceFile(string $sourceFile)
    {
        return $this->telemetryRepository->getTelemetryBySourceFile($sourceFile);
    }
}
