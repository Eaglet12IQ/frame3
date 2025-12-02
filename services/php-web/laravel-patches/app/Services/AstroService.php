<?php

namespace App\Services;

use App\Clients\AstroClient;

class AstroService
{
    private AstroClient $client;

    public function __construct(AstroClient $client)
    {
        $this->client = $client;
    }

    public function getEvents(array $params = []): array
    {
        $lat = (float)($params['lat'] ?? 65.9558);
        $lon = (float)($params['lon'] ?? 37.6171);
        $elev = (float)($params['elevation'] ?? 7);
        $from = $params['from_date'] ?? '2025-11-29';
        $to = $params['to_date'] ?? '2026-11-29';
        $time = $params['time'] ?? '00:00:00';
        $output = $params['output'] ?? 'rows';

        $rows = $this->client->getEvents([
            'lat' => $lat,
            'lon' => $lon,
            'elevation' => $elev,
            'from_date' => $from,
            'to_date' => $to,
            'time' => $time,
            'output' => $output,
        ]);

        return [
            'data' => [
                'dates' => [
                    'from' => $from . 'T' . $time . '.000+03:00',
                    'to' => $to . 'T' . $time . '.000+03:00'
                ],
                'observer' => [
                    'location' => [
                        'longitude' => $lon,
                        'latitude' => $lat,
                        'elevation' => $elev
                    ]
                ],
                'rows' => $rows
            ]
        ];
    }
}
