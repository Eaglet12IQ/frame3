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
        return $this->cmsPageRepository->getPageContent($slug);
    }
}
