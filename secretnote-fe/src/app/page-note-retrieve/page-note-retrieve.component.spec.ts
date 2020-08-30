import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { PageNoteRetrieveComponent } from './page-note-retrieve.component';

describe('PageNoteRetrieveComponent', () => {
  let component: PageNoteRetrieveComponent;
  let fixture: ComponentFixture<PageNoteRetrieveComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ PageNoteRetrieveComponent ]
    })
    .compileComponents();
  }));

  beforeEach(() => {
    fixture = TestBed.createComponent(PageNoteRetrieveComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
