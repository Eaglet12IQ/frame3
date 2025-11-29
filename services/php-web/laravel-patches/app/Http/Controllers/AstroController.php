<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;

class AstroController extends Controller
{
    public function index()
    {
        return view('astro', ['events' => null]);
    }

    public function events(Request $r)
    {
        $lat   = (float) $r->query('lat', 65.9558);
        $lon   = (float) $r->query('lon', 37.6171);
        $elev  = (float) $r->query('elevation', 7);      // обязательное поле
        $from  = $r->query('from_date', '2025-11-29');
        $to    = $r->query('to_date', '2026-11-29');
        $time  = $r->query('time', '00:00:00');          // обязательное поле
        $output = $r->query('output', 'rows');         // 'table' или 'rows'

        $appId  = env('ASTRO_APP_ID');
        $secret = env('ASTRO_APP_SECRET');
        if (!$appId || !$secret) {
            return response()->json(['error' => 'Missing ASTRO_APP_ID or ASTRO_APP_SECRET'], 500);
        }

        $auth = base64_encode($appId . ':' . $secret);

        $bodies = ['sun', 'moon'];
        $rows = [];

        foreach ($bodies as $body) {
            $url = 'https://api.astronomyapi.com/api/v2/bodies/events/' . urlencode($body) . '?' . http_build_query([
                'latitude'  => $lat,
                'longitude' => $lon,
                'elevation' => $elev,
                'from_date' => $from,
                'to_date'   => $to,
                'time'      => $time,
                'output'    => $output,
            ]);

            $ch = curl_init($url);
            curl_setopt_array($ch, [
                CURLOPT_RETURNTRANSFER => true,
                CURLOPT_HTTPHEADER     => [
                    'Authorization: Basic ' . $auth,
                    'Content-Type: application/json',
                ],
                CURLOPT_TIMEOUT        => 20,
            ]);

            $raw  = curl_exec($ch);
            $code = curl_getinfo($ch, CURLINFO_RESPONSE_CODE) ?: 0;
            $err  = curl_error($ch);
            curl_close($ch);

            if ($raw === false || $code >= 400) {
                return response()->json([
                    'error' => $err ?: ("HTTP " . $code),
                    'code'  => $code,
                    'raw'   => $raw
                ], $code ?: 500);
            }

            $data = json_decode($raw, true);
            if (isset($data['data']['rows'])) {
                $rows = array_merge($rows, $data['data']['rows']);
            }
        }

        $response = [
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

        return response()->json($response);
    }
}
