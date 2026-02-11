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
        Schema::create('kafka_messages', function (Blueprint $table) {
            $table->id();
            $table->string('topic', 100);
            $table->string('key', 255)->nullable();
            $table->longText('value');
            $table->boolean('consumed')->default(false);
            $table->timestamp('consumed_at')->nullable();
            $table->timestamps();
            
            $table->index('topic');
            $table->index('consumed');
            $table->index('created_at');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('kafka_messages');
    }
};
