import { ComponentFixture, TestBed, waitForAsync } from '@angular/core/testing';

import {PageChatCreateComponent} from './page-chat-create.component';
import {RouterTestingModule} from "@angular/router/testing";

describe('PageChatCreateComponent', () => {
    let component: PageChatCreateComponent;
    let fixture: ComponentFixture<PageChatCreateComponent>;

    beforeEach(waitForAsync(() => {
        TestBed.configureTestingModule({
            declarations: [PageChatCreateComponent],
            imports: [RouterTestingModule]
        })
            .compileComponents();
    }));

    beforeEach(() => {
        fixture = TestBed.createComponent(PageChatCreateComponent);
        component = fixture.componentInstance;
        fixture.detectChanges();
    });

    it('should create', () => {
        expect(component).toBeTruthy();
    });
});
