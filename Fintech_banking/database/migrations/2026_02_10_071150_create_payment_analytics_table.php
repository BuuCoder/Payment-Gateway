<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('payment_analytics', function (Blueprint $table) {
            $table->id();
            $table->unsignedBigInteger('payment_id')->index();
            $table->unsignedBigInteger('user_id')->index();
            $table->decimal('amount', 15, 2);
            $table->string('currency', 3)->index();
            $table->enum('status', ['PENDING', 'PROCESSING', 'SUCCESS', 'FAILED', 'FRAUD_DETECTED'])->index();
            $table->string('payment_method', 50)->index();
            $table->string('merchant_id', 100)->nullable()->index();
            $table->string('error_code', 50)->nullable();
            $table->text('error_message')->nullable();
            $table->integer('retry_count')->default(0);
            $table->timestamp('processed_at')->nullable()->index();
            $table->timestamp('event_timestamp')->nullable()->index();
            $table->timestamps();

            // Indexes for analytics queries
            $table->index(['status', 'created_at']);
            $table->index(['currency', 'status']);
            $table->index(['payment_method', 'status']);
            $table->index(['user_id', 'created_at']);
            
            // Allow multiple events per payment (for retries)
            // Each status change creates a new analytics record
            $table->index(['payment_id', 'status', 'created_at']);
        });

        // Hourly aggregation table for fast queries
        Schema::create('payment_hourly_stats', function (Blueprint $table) {
            $table->id();
            $table->date('date')->index();
            $table->integer('hour')->index();
            $table->string('currency', 3)->index();
            $table->string('status', 20)->index();
            $table->integer('count')->default(0);
            $table->decimal('total_amount', 15, 2)->default(0);
            $table->decimal('avg_amount', 15, 2)->default(0);
            $table->timestamps();

            $table->unique(['date', 'hour', 'currency', 'status']);
        });

        // Daily aggregation table
        Schema::create('payment_daily_stats', function (Blueprint $table) {
            $table->id();
            $table->date('date')->index();
            $table->string('currency', 3)->index();
            $table->string('status', 20)->index();
            $table->integer('count')->default(0);
            $table->decimal('total_amount', 15, 2)->default(0);
            $table->decimal('avg_amount', 15, 2)->default(0);
            $table->decimal('min_amount', 15, 2)->default(0);
            $table->decimal('max_amount', 15, 2)->default(0);
            $table->timestamps();

            $table->unique(['date', 'currency', 'status']);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('payment_analytics');
        Schema::dropIfExists('payment_hourly_stats');
        Schema::dropIfExists('payment_daily_stats');
    }
};
