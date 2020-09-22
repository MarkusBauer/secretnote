import {browser, by, element} from 'protractor';

export class AppPage {
    navigateTo(): Promise<unknown> {
        return browser.get(browser.baseUrl) as Promise<unknown>;
    }

    getNavbarBrandText(): Promise<string> {
        return element(by.css('app-root nav .navbar-brand')).getText() as Promise<string>;
    }
}
