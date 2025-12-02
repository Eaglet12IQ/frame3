<?php

namespace App\Repositories;

use App\Models\CmsBlock;

class CmsBlockRepository extends BaseRepository
{
    public function __construct(CmsBlock $model)
    {
        parent::__construct($model);
    }

    public function getActiveBlocks()
    {
        return $this->model->where('is_active', true)->get();
    }

    public function findBySlug($slug): ?CmsBlock
    {
        return $this->model->where('slug', $slug)->first();
    }

    public function getActiveBlockBySlug($slug): ?CmsBlock
    {
        return $this->model->where('slug', $slug)->where('is_active', true)->first();
    }
}
