<?php

namespace App\Repositories;

use App\Models\CmsPage;

class CmsPageRepository extends BaseRepository
{
    public function __construct(CmsPage $model)
    {
        parent::__construct($model);
    }

    public function findBySlug(string $slug): ?CmsPage
    {
        return $this->model->where('slug', $slug)->first();
    }

    public function getPageContent(string $slug): ?object
    {
        $page = $this->findBySlug($slug);
        return $page ? (object)['title' => $page->title, 'body' => $page->body] : null;
    }
}
