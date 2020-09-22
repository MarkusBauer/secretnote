import {async, ComponentFixture, TestBed} from '@angular/core/testing';

import {PageNoteStoreComponent} from './page-note-store.component';
import {HttpClientTestingModule} from "@angular/common/http/testing";
import {RouterTestingModule} from "@angular/router/testing";

describe('PageNoteStoreComponent', () => {
    let component: PageNoteStoreComponent;
    let fixture: ComponentFixture<PageNoteStoreComponent>;

    beforeEach(async(() => {
        TestBed.configureTestingModule({
            declarations: [PageNoteStoreComponent],
            imports: [HttpClientTestingModule, RouterTestingModule],
        }).compileComponents();
    }));

    beforeEach(() => {
        fixture = TestBed.createComponent(PageNoteStoreComponent);
        component = fixture.componentInstance;
        fixture.detectChanges();
    });

    it('should create', () => {
        expect(component).toBeTruthy();
    });
});
