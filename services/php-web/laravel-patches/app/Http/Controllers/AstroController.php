<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\AstroService;

class AstroController extends Controller
{
    protected $astroService;

    public function __construct(AstroService $astroService)
    {
        $this->astroService = $astroService;
    }

    public function index()
    {
        return view('astro', ['events' => null]);
    }

    public function events(Request $r)
    {
        try {
            $validated = $r->validate([
                'lat' => 'nullable|numeric',
                'lon' => 'nullable|numeric',
                'elevation' => 'nullable|numeric',
                'from_date' => 'nullable|date',
                'to_date' => 'nullable|date',
                'time' => 'nullable|string',
            ]);

            $dto = $this->astroService->getEvents($validated);
            return response()->json($dto->toArray());
        } catch (\Exception $e) {
            return $this->errorResponse($e->getMessage(), $e->getCode() ?: 500);
        }
    }
}
