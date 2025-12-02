<?php

namespace App\Http\Controllers;

use App\Repositories\CmsPageRepository;

class CmsController extends Controller
{
    protected $cmsPageRepository;

    public function __construct(CmsPageRepository $cmsPageRepository)
    {
        $this->cmsPageRepository = $cmsPageRepository;
    }

    public function page(string $slug)
    {
        $row = $this->cmsPageRepository->getPageContent($slug);
        if (!$row) abort(404);
        return response()->view('cms.page', ['title' => $row->title, 'html' => $row->body]);
    }
}
