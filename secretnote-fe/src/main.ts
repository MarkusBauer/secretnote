// Not so stylish warning for unsupported browsers - IE<11 for now
if (/MSIE |Trident\//.test(window.navigator.userAgent) && !/ rv:11|Edge\//.test(window.navigator.userAgent)) {
    document.body.innerHTML += '<div class="container"><div class="card text-white bg-danger mt-3">' +
        '<div class="card-header">Unsupported browser</div>' +
        '<div class="card-body"><p class="card-text">Your browser is outdated and not supported. It will likely not work. Please switch to a recent version of Firefox or Chrome.</p></div>' +
        '</div></div>';
}

import {enableProdMode} from '@angular/core';
import {platformBrowserDynamic} from '@angular/platform-browser-dynamic';

import {AppModule} from './app/app.module';
import {environment} from './environments/environment';

if (environment.production) {
    enableProdMode();
}

document.addEventListener('DOMContentLoaded', () => {
    platformBrowserDynamic().bootstrapModule(AppModule)
        .catch(err => console.error(err));
});
