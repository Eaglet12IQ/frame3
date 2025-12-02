<?php

use Illuminate\Support\Facades\Route;

Route::middleware([\App\Http\Middleware\LoggingMiddleware::class, \App\Http\Middleware\RateLimitingMiddleware::class])->group(function () {
    Route::get('/', fn() => redirect('/dashboard'));

    // Панели
    Route::get('/dashboard', [\App\Http\Controllers\DashboardController::class, 'index']);
    Route::get('/dashboard/download-telemetry-csv', [\App\Http\Controllers\DashboardController::class, 'downloadTelemetryCsv'])->name('dashboard.download-telemetry-csv');
    Route::get('/iss',       [\App\Http\Controllers\IssController::class,       'index']);
    Route::get('/jwst',      [\App\Http\Controllers\JwstController::class,      'index']);
    Route::get('/osdr',      [\App\Http\Controllers\OsdrController::class,      'index']);
    Route::get('/astro',     [\App\Http\Controllers\AstroController::class,     'index']);

    // ISS API endpoints
    Route::get('/api/iss/last',  [\App\Http\Controllers\IssController::class, 'last']);
    Route::get('/api/iss/trend', [\App\Http\Controllers\IssController::class, 'trend']);

    // JWST галерея (JSON)
    Route::get('/api/jwst/feed', [\App\Http\Controllers\JwstController::class, 'feed']);
    Route::get("/api/astro/events", [\App\Http\Controllers\AstroController::class, "events"]);
    Route::get('/page/{slug}', [\App\Http\Controllers\CmsController::class, 'page']);
});
