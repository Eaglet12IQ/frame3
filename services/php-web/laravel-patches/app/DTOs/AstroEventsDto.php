<?php

namespace App\DTOs;

class AstroEventsDto
{
    public array $dates;
    public array $observer;
    public array $rows;

    public function __construct(array $dates, array $observer, array $rows)
    {
        $this->dates = $dates;
        $this->observer = $observer;
        $this->rows = $rows;
    }

    public function toArray(): array
    {
        return [
            'data' => [
                'dates' => $this->dates,
                'observer' => $this->observer,
                'rows' => $this->rows,
            ],
        ];
    }
}
