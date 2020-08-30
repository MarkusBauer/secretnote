import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { PageNoteStoreComponent } from './page-note-store.component';

describe('PageNoteStoreComponent', () => {
  let component: PageNoteStoreComponent;
  let fixture: ComponentFixture<PageNoteStoreComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ PageNoteStoreComponent ]
    })
    .compileComponents();
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
