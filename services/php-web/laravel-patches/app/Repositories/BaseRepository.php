<?php

namespace App\Repositories;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Collection;

abstract class BaseRepository
{
    protected $model;

    public function __construct(Model $model)
    {
        $this->model = $model;
    }

    public function all(): Collection
    {
        return $this->model->all();
    }

    public function find($id): ?Model
    {
        return $this->model->find($id);
    }

    public function create(array $attributes): Model
    {
        return $this->model->create($attributes);
    }

    public function update($id, array $attributes): bool
    {
        $model = $this->find($id);
        return $model ? $model->update($attributes) : false;
    }

    public function delete($id): bool
    {
        $model = $this->find($id);
        return $model ? $model->delete() : false;
    }

    public function paginate($perPage = 15)
    {
        return $this->model->paginate($perPage);
    }

    public function where(array $conditions)
    {
        return $this->model->where($conditions);
    }
}
