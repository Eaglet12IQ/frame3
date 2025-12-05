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

    public function getLatestTelemetryPaginated(int $perPage = 10)
    {
        return $this->telemetryRepository->getLatestTelemetryPaginated($perPage);
    }

    public function getTelemetryBySourceFile(string $sourceFile)
    {
        return $this->telemetryRepository->getTelemetryBySourceFile($sourceFile);
    }

    public function getTelemetryBySourceFilePaginated(string $sourceFile, int $perPage = 100)
    {
        return $this->telemetryRepository->getTelemetryBySourceFilePaginated($sourceFile, $perPage);
    }
}
