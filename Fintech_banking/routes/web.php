<?php

use App\Http\Controllers\PaymentViewController;
use Illuminate\Support\Facades\Route;

Route::get('/', function () {
    return redirect()->route('payments.index');
});

// Payment views
Route::get('/payments', [PaymentViewController::class, 'index'])->name('payments.index');
Route::get('/payments/create', [PaymentViewController::class, 'create'])->name('payments.create');
Route::get('/payments/{id}', [PaymentViewController::class, 'show'])->name('payments.show');

// Load testing dashboard
Route::get('/load-test', [PaymentViewController::class, 'loadTest'])->name('load-test');

// Performance dashboard
Route::get('/dashboard', [PaymentViewController::class, 'dashboard'])->name('dashboard');
