import { ComponentFixture, TestBed, waitForAsync } from '@angular/core/testing';

import {PageNoteRetrieveComponent} from './page-note-retrieve.component';
import {RouterTestingModule} from "@angular/router/testing";
import {HttpClientTestingModule} from "@angular/common/http/testing";

describe('PageNoteRetrieveComponent', () => {
    let component: PageNoteRetrieveComponent;
    let fixture: ComponentFixture<PageNoteRetrieveComponent>;
    let router: RouterTestingModule;

    beforeEach(waitForAsync(() => {
        TestBed.configureTestingModule({
            declarations: [PageNoteRetrieveComponent],
            imports: [RouterTestingModule.withRoutes([]), HttpClientTestingModule]
        }).compileComponents();
        router = TestBed.inject(RouterTestingModule);
    }));

    beforeEach(() => {
        router
        fixture = TestBed.createComponent(PageNoteRetrieveComponent);
        component = fixture.componentInstance;
        fixture.detectChanges();
    });

    it('should create', () => {
        expect(component).toBeTruthy();
    });
});
