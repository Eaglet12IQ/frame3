<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;

class RateLimitingMiddleware
{
    public function handle(Request $request, Closure $next)
    {
        $ip = $request->ip();
        $key = "rate_limit:{$ip}";
        $maxAttempts = 100; // Max requests per minute
        $decayMinutes = 1;

        $attempts = Cache::get($key, 0);

        if ($attempts >= $maxAttempts) {
            return response()->json([
                'error' => 'Too many requests. Please try again later.',
                'code' => 429
            ], 429);
        }

        Cache::put($key, $attempts + 1, $decayMinutes * 60);

        return $next($request);
    }
}
