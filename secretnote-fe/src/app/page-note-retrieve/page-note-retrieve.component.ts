import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from "@angular/router";

@Component({
    selector: 'app-page-note-retrieve',
    templateUrl: './page-note-retrieve.component.html',
    styleUrls: ['./page-note-retrieve.component.less']
})
export class PageNoteRetrieveComponent implements OnInit {

    ident: string;
    key: string;
    validIdent: boolean = false;

    constructor(private route: ActivatedRoute) {
    }

    ngOnInit() {
        this.route.paramMap.subscribe(map => {
            this.ident = map.get("ident");
            this.validIdent = this.updateIdentKey();
        });
        this.route.fragment.subscribe(f => {
            this.key = f;
            this.validIdent = this.updateIdentKey();
        })
    }

    updateIdentKey() {
        if (!this.ident || this.ident.length != 24) {
            // TODO report error
            return false;
        }
        // TODO check key

        return true;
    }

}
