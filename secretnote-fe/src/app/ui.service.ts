import {Injectable, Output, EventEmitter} from '@angular/core';
import {NavigationEnd, Router} from "@angular/router";


interface Alert {
    type: string;
    message: string;
}

@Injectable({
    providedIn: 'root'
})
export class UiService {

    alerts: Array<Alert> = [];

    navbarCollapsed = true;

    @Output() navbarCollapseEvents = new EventEmitter<boolean>();


    constructor(private router: Router) {
        this.router.events.subscribe(event => {
            if (event instanceof NavigationEnd) {
                if (!this.navbarCollapsed) this.navbarCollapsed = true;
            }
        });
    }

    close(alert: Alert) {
        this.alerts.splice(this.alerts.indexOf(alert), 1);
    }

    alert(alert: Alert) {
        this.alerts.push(alert);
    }

    error(message: string) {
        this.alert({type: "danger", message: message});
    }

    toggleNavbar() {
        this.navbarCollapsed = !this.navbarCollapsed;
        this.navbarCollapseEvents.emit(this.navbarCollapsed);
    }
}
