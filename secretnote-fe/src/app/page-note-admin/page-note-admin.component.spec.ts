import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { PageNoteAdminComponent } from './page-note-admin.component';

describe('PageNoteAdminComponent', () => {
  let component: PageNoteAdminComponent;
  let fixture: ComponentFixture<PageNoteAdminComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ PageNoteAdminComponent ]
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
