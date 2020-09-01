import { TestBed } from '@angular/core/testing';

import { ChatJoinGuard } from './chat-join.guard';

describe('ChatJoinGuard', () => {
  let guard: ChatJoinGuard;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    guard = TestBed.inject(ChatJoinGuard);
  });

  it('should be created', () => {
    expect(guard).toBeTruthy();
  });
});
