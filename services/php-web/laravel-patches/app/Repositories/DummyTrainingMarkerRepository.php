<?php

namespace App\Repositories;

use App\Models\DummyTrainingMarker;

class DummyTrainingMarkerRepository extends BaseRepository
{
    public function __construct(DummyTrainingMarker $model)
    {
        parent::__construct($model);
    }

    public function getMarkersWithNotes()
    {
        return $this->model->whereNotNull('note')->get();
    }

    public function getRecentMarkers($limit = 10)
    {
        return $this->model->orderBy('created_at', 'desc')->limit($limit)->get();
    }
}
