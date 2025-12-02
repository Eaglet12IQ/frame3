<?php

namespace App\Services;

class AstroService
{
    public function getEvents(array $params = []): array
    {
        $lat = (float)($params['lat'] ?? 65.9558);
        $lon = (float)($params['lon'] ?? 37.6171);
        $elev = (float)($params['elevation'] ?? 7);
        $from = $params['from_date'] ?? '2025-11-29';
        $to = $params['to_date'] ?? '2026-11-29';
        $time = $params['time'] ?? '00:00:00';
        $output = $params['output'] ?? 'rows';

        $appId = env('ASTRO_APP_ID');
        $secret = env('ASTRO_APP_SECRET');
        if (!$appId || !$secret) {
            throw new \Exception('Missing ASTRO_APP_ID or ASTRO_APP_SECRET');
        }

        $auth = base64_encode($appId . ':' . $secret);

        $bodies = ['sun', 'moon'];
        $rows = [];

        foreach ($bodies as $body) {
            $url = 'https://api.astronomyapi.com/api/v2/bodies/events/' . urlencode($body) . '?' . http_build_query([
                'latitude' => $lat,
                'longitude' => $lon,
                'elevation' => $elev,
                'from_date' => $from,
                'to_date' => $to,
                'time' => $time,
                'output' => $output,
            ]);

            $ch = curl_init($url);
            curl_setopt_array($ch, [
                CURLOPT_RETURNTRANSFER => true,
                CURLOPT_HTTPHEADER => [
                    'Authorization: Basic ' . $auth,
                    'Content-Type: application/json',
                ],
                CURLOPT_TIMEOUT => 20,
            ]);

            $raw = curl_exec($ch);
            $code = curl_getinfo($ch, CURLINFO_RESPONSE_CODE) ?: 0;
            $err = curl_error($ch);
            curl_close($ch);

            if ($raw === false || $code >= 400) {
                throw new \Exception($err ?: ("HTTP " . $code), $code ?: 500);
            }

            $data = json_decode($raw, true);
            if (isset($data['data']['rows'])) {
                $rows = array_merge($rows, $data['data']['rows']);
            }
        }

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
