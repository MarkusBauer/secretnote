import {Component} from '@angular/core';
import {UiService} from "./ui.service";

@Component({
    selector: 'app-root',
    templateUrl: './app.component.html',
    styleUrls: ['./app.component.less']
})
export class AppComponent {
    title = 'secretnote';

    constructor(public ui: UiService) {
    }
}
