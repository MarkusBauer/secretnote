import {Component} from '@angular/core';
import {UiService} from "./ui.service";
import {environment} from '../environments/environment';

@Component({
    selector: 'app-root',
    templateUrl: './app.component.html',
    styleUrls: ['./app.component.less']
})
export class AppComponent {
    title = 'SecretNote';

    environment = environment;

    navbarCollapsed = true;

    constructor(public ui: UiService) {
    }

    toggleNavbar() {
        this.navbarCollapsed = !this.navbarCollapsed;
    }
}
