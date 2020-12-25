import {Component, OnInit} from '@angular/core';
import {environment} from "../../environments/environment";

@Component({
    selector: 'app-page-faq',
    templateUrl: './page-faq.component.html',
    styleUrls: ['./page-faq.component.less']
})
export class PageFaqComponent implements OnInit {

    environment = environment;

    constructor() {
    }

    ngOnInit(): void {
    }

}
