import { TestBed } from '@angular/core/testing';

import { UsernamesService } from './usernames.service';

describe('UsernamesService', () => {
  let service: UsernamesService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(UsernamesService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
