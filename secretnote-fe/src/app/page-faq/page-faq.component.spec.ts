import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { PageFaqComponent } from './page-faq.component';

describe('PageFaqComponent', () => {
  let component: PageFaqComponent;
  let fixture: ComponentFixture<PageFaqComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ PageFaqComponent ]
    })
    .compileComponents();
  }));

  beforeEach(() => {
    fixture = TestBed.createComponent(PageFaqComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
