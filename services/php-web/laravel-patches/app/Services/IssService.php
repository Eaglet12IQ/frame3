<?php

namespace App\Services;

use App\Clients\RustClient;

class IssService
{
    private RustClient $client;

    public function __construct(RustClient $client)
    {
        $this->client = $client;
    }

    public function getLast(): array
    {
        return $this->client->getJson('last');
    }

    public function getTrend(int $limit = 240): array
    {
        return $this->client->getJson('iss/trend', ['limit' => $limit]);
    }

    public function getTrendAnalysis(): array
    {
        return $this->client->getJson('iss/trend/analysis');
    }
}
