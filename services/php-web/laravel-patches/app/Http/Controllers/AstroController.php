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
                'date' => 'nullable|date',
                'limit' => 'nullable|integer|min:1|max:100',
                'type' => 'nullable|string|max:50',
            ]);

            $dto = $this->astroService->getEvents($validated);
            return response()->json($dto->toArray());
        } catch (\Exception $e) {
            return $this->errorResponse($e->getMessage(), $e->getCode() ?: 500);
        }
    }
}
