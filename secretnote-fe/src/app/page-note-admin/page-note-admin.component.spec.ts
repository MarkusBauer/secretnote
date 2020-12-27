import { ComponentFixture, TestBed, waitForAsync } from '@angular/core/testing';

import {PageNoteAdminComponent} from './page-note-admin.component';
import {RouterTestingModule} from "@angular/router/testing";
import {HttpClientTestingModule} from "@angular/common/http/testing";

describe('PageNoteAdminComponent', () => {
    let component: PageNoteAdminComponent;
    let fixture: ComponentFixture<PageNoteAdminComponent>;

    beforeEach(waitForAsync(() => {
        TestBed.configureTestingModule({
            declarations: [PageNoteAdminComponent],
            imports: [RouterTestingModule, HttpClientTestingModule]
        })
            .compileComponents();
    }));

    beforeEach(() => {
        fixture = TestBed.createComponent(PageNoteAdminComponent);
        component = fixture.componentInstance;
        fixture.detectChanges();
    });

    it('should create', () => {
        expect(component).toBeTruthy();
    });
});
