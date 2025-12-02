<?php

namespace App\Repositories;

use App\Models\Telemetry;

class TelemetryRepository extends BaseRepository
{
    public function __construct(Telemetry $model)
    {
        parent::__construct($model);
    }

    public function getLatestTelemetry(int $limit = 10)
    {
        return $this->model
            ->select('recorded_at', 'voltage', 'temp', 'source_file')
            ->orderBy('recorded_at', 'desc')
            ->limit($limit)
            ->get();
    }
}
