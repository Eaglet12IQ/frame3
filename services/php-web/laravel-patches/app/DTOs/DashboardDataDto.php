<?php

namespace App\DTOs;

class DashboardDataDto
{
    public array $iss;
    public array $trend;
    public array $jw_gallery;
    public array $jw_observation_raw;
    public array $jw_observation_summary;
    public array $jw_observation_images;
    public array $jw_observation_files;
    public array $metrics;
    public array $telemetry;
    public $cms_block;

    public function __construct(
        array $iss,
        array $trend,
        array $jw_gallery,
        array $jw_observation_raw,
        array $jw_observation_summary,
        array $jw_observation_images,
        array $jw_observation_files,
        array $metrics,
        array $telemetry,
        $cms_block
    ) {
        $this->iss = $iss;
        $this->trend = $trend;
        $this->jw_gallery = $jw_gallery;
        $this->jw_observation_raw = $jw_observation_raw;
        $this->jw_observation_summary = $jw_observation_summary;
        $this->jw_observation_images = $jw_observation_images;
        $this->jw_observation_files = $jw_observation_files;
        $this->metrics = $metrics;
        $this->telemetry = $telemetry;
        $this->cms_block = $cms_block;
    }

    public function toArray(): array
    {
        return [
            'iss' => $this->iss,
            'trend' => $this->trend,
            'jw_gallery' => $this->jw_gallery,
            'jw_observation_raw' => $this->jw_observation_raw,
            'jw_observation_summary' => $this->jw_observation_summary,
            'jw_observation_images' => $this->jw_observation_images,
            'jw_observation_files' => $this->jw_observation_files,
            'metrics' => $this->metrics,
            'telemetry' => $this->telemetry,
            'cms_block' => $this->cms_block,
        ];
    }
}
