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
            $params = $r->query();
            $dto = $this->astroService->getEvents($params);
            return response()->json($dto->toArray());
        } catch (\Exception $e) {
            return response()->json([
                'error' => $e->getMessage(),
                'code' => $e->getCode() ?: 500
            ], $e->getCode() ?: 500);
        }
    }
}
