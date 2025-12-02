<?php

namespace App\DTOs;

class JwstFeedDto
{
    public string $source;
    public int $count;
    public array $items;

    public function __construct(string $source, int $count, array $items)
    {
        $this->source = $source;
        $this->count = $count;
        $this->items = $items;
    }

    public function toArray(): array
    {
        return [
            'source' => $this->source,
            'count' => $this->count,
            'items' => $this->items,
        ];
    }
}
