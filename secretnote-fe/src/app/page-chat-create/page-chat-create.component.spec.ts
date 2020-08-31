import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { PageChatCreateComponent } from './page-chat-create.component';

describe('PageChatCreateComponent', () => {
  let component: PageChatCreateComponent;
  let fixture: ComponentFixture<PageChatCreateComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ PageChatCreateComponent ]
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
