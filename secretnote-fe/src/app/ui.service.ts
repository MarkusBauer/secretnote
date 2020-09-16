import {Injectable, Output, EventEmitter} from '@angular/core';
import {NavigationEnd, Router} from "@angular/router";


interface Alert {
    type: string;
    class: string;
    message: string;
}

@Injectable({
    providedIn: 'root'
})
export class UiService {

    alerts: Array<Alert> = [];

    navbarCollapsed = true;

    @Output() navbarCollapseEvents = new EventEmitter<boolean>();

    httpErrorHandler = (error) => {
        console.error(error);
        this.error("Server error: "+error, {header: "Operation failed"});
    };


    constructor(private router: Router) {
        this.router.events.subscribe(event => {
            if (event instanceof NavigationEnd) {
                if (!this.navbarCollapsed) this.navbarCollapsed = true;
            }
        });
    }

    close(alert: Alert) {
        // this.alerts.splice(this.alerts.indexOf(alert), 1);
        this.alerts = this.alerts.filter(a => a != alert);
    }

    alert(alert: Alert) {
        this.alerts.push(alert);
    }

    error(message: string, options = {}) {
        this.alert({type: "danger", class: "bg-danger text-white", message: message, ...options});
    }

    warning(message: string, options = {}) {
        this.alert({type: "warning", class: "bg-warning", message: message, ...options});
    }

    success(message: string, options = {}) {
        this.alert({type: "success", class: "bg-success text-white", message: message, ...options});
    }

    toggleNavbar() {
        this.navbarCollapsed = !this.navbarCollapsed;
        this.navbarCollapseEvents.emit(this.navbarCollapsed);
    }
}
