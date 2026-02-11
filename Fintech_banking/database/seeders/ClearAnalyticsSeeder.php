<?php

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;

class ClearAnalyticsSeeder extends Seeder
{
    /**
     * Clear analytics tables to fix duplicate issue
     */
    public function run(): void
    {
        echo "Clearing analytics tables...\n";
        
        DB::table('payment_analytics')->truncate();
        DB::table('payment_hourly_stats')->truncate();
        DB::table('payment_daily_stats')->truncate();
        
        echo "Analytics tables cleared successfully!\n";
    }
}
