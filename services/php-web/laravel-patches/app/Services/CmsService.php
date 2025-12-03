<?php

namespace App\Services;

use App\Repositories\CmsPageRepository;

class CmsService
{
    protected $cmsPageRepository;

    public function __construct(CmsPageRepository $cmsPageRepository)
    {
        $this->cmsPageRepository = $cmsPageRepository;
    }

    public function getPageContent(string $slug)
    {
        $page = $this->cmsPageRepository->getPageContent($slug);
        if ($page) {
            // Sanitize HTML content to prevent XSS
            $page->body = $this->sanitizeHtml($page->body);
        }
        return $page;
    }

    private function sanitizeHtml(string $html): string
    {
        // Allow basic HTML tags, remove potentially dangerous ones
        $allowedTags = '<p><br><strong><em><u><h1><h2><h3><h4><h5><h6><ul><ol><li><a><img>';
        return strip_tags($html, $allowedTags);
    }
}
