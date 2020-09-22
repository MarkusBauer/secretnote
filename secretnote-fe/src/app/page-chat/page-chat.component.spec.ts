import {async, ComponentFixture, TestBed} from '@angular/core/testing';

import {PageChatComponent} from './page-chat.component';
import {RouterTestingModule} from "@angular/router/testing";
import {HttpClientTestingModule} from "@angular/common/http/testing";

describe('PageChatComponent', () => {
    let component: PageChatComponent;
    let fixture: ComponentFixture<PageChatComponent>;

    beforeEach(async(() => {
        TestBed.configureTestingModule({
            declarations: [PageChatComponent],
            imports: [RouterTestingModule, HttpClientTestingModule]
        })
            .compileComponents();
    }));

    beforeEach(() => {
        fixture = TestBed.createComponent(PageChatComponent);
        component = fixture.componentInstance;
        fixture.detectChanges();
    });

    it('should create', () => {
        expect(component).toBeTruthy();
    });
});
