<?php

namespace App\Services;

use App\Repositories\CmsBlockRepository;
use App\DTOs\DashboardDataDto;

class DashboardService
{
    protected $issService;
    protected $telemetryService;
    protected $cmsBlockRepository;

    public function __construct(
        IssService $issService,
        TelemetryService $telemetryService,
        CmsBlockRepository $cmsBlockRepository
    ) {
        $this->issService = $issService;
        $this->telemetryService = $telemetryService;
        $this->cmsBlockRepository = $cmsBlockRepository;
    }

    public function getDashboardData(): DashboardDataDto
    {
        $iss = $this->issService->getLast();
        $telemetry = $this->telemetryService->getLatestTelemetry(10)->toArray();
        $cmsBlock = $this->cmsBlockRepository->getActiveBlockBySlug('dashboard_experiment');

        // Sanitize CMS block content to prevent XSS
        if ($cmsBlock) {
            $cmsBlock->content = $this->sanitizeHtml($cmsBlock->content);
        }

        $trend = []; // Front-end will fetch /api/iss/trend via nginx proxy
        $jw_gallery = []; // Not needed server-side
        $jw_observation_raw = [];
        $jw_observation_summary = [];
        $jw_observation_images = [];
        $jw_observation_files = [];
        $metrics = [
            'iss_speed' => $iss['payload']['velocity'] ?? null,
            'iss_alt' => $iss['payload']['altitude'] ?? null,
            'neo_total' => 0,
        ];

        return new DashboardDataDto(
            $iss,
            $trend,
            $jw_gallery,
            $jw_observation_raw,
            $jw_observation_summary,
            $jw_observation_images,
            $jw_observation_files,
            $metrics,
            $telemetry,
            $cmsBlock
        );
    }

    private function sanitizeHtml(string $html): string
    {
        // Allow basic HTML tags, remove potentially dangerous ones
        $allowedTags = '<p><br><strong><em><u><h1><h2><h3><h4><h5><h6><ul><ol><li><a><img>';
        return strip_tags($html, $allowedTags);
    }
}
