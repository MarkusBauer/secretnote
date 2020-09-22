import {TestBed} from '@angular/core/testing';

import {ChatJoinGuard} from './chat-join.guard';
import {RouterTestingModule} from "@angular/router/testing";
import {HttpClientTestingModule} from "@angular/common/http/testing";

describe('ChatJoinGuard', () => {
    let guard: ChatJoinGuard;

    beforeEach(() => {
        TestBed.configureTestingModule({
            imports: [RouterTestingModule, HttpClientTestingModule]
        });
        guard = TestBed.inject(ChatJoinGuard);
    });

    it('should be created', () => {
        expect(guard).toBeTruthy();
    });
});
