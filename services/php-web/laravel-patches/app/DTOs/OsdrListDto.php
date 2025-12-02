<?php

namespace App\DTOs;

class OsdrListDto
{
    public array $items;
    public string $src;

    public function __construct(array $items, string $src)
    {
        $this->items = $items;
        $this->src = $src;
    }

    public function toArray(): array
    {
        return [
            'items' => $this->items,
            'src' => $this->src,
        ];
    }
}
