<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\DashboardService;

class DashboardController extends Controller
{
    protected $dashboardService;

    public function __construct(DashboardService $dashboardService)
    {
        $this->dashboardService = $dashboardService;
    }

    public function index()
    {
        $dto = $this->dashboardService->getDashboardData();
        $data = $dto->toArray();

        return view('dashboard', [
            'cms_block' => $data['cms_block'],
            'telemetry' => $data['telemetry'],
        ]);
    }

    public function data()
    {
        $dto = $this->dashboardService->getDashboardData();
        return response()->json($dto->toArray());
    }
}
