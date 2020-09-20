import {Component} from '@angular/core';
import {UiService} from "./ui.service";
import {environment} from '../environments/environment';
import {Router} from "@angular/router";

@Component({
    selector: 'app-root',
    templateUrl: './app.component.html',
    styleUrls: ['./app.component.less']
})
export class AppComponent {
    title = 'SecretNote';

    environment = environment;

    constructor(public ui: UiService, public router: Router) {
    }
}
