<?php

namespace App\Http\Controllers;

use App\Services\CmsService;

class CmsController extends Controller
{
    protected $cmsService;

    public function __construct(CmsService $cmsService)
    {
        $this->cmsService = $cmsService;
    }

    public function page(string $slug)
    {
        $row = $this->cmsService->getPageContent($slug);
        if (!$row) abort(404);
        return response()->view('cms.page', ['title' => $row->title, 'html' => $row->body]);
    }
}
