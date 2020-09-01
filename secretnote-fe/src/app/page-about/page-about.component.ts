import {Component, OnInit} from '@angular/core';
import {environment} from "../../environments/environment";

@Component({
    selector: 'app-page-about',
    templateUrl: './page-about.component.html',
    styleUrls: ['./page-about.component.less']
})
export class PageAboutComponent implements OnInit {

    environment = environment;

    constructor() {
    }

    ngOnInit(): void {
    }

}
