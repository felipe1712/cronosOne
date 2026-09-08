"use client";

import React from "react";
import Link from "next/link";
import { Button, Toolbar, IconButton, Box } from "@mui/material";
import { usePathname } from "next/navigation";

const HorizontalNavbar = () => {
  const pathname = usePathname();

  return (
    <>
      <div className="horizontal-navbar-area">
        <div className="accordion">
          {/* Max 7 accordion-item in this list */}
          <div className="accordion-item megamenu">
            <Button type="button" className="accordion-button">
              <i className="material-symbols-outlined">dashboard</i>
              <span className="title" style={{ lineHeight: 1 }}>
                Dashboard
              </span>
              <span className="trezo-badge">25</span>
            </Button>
            <div className="accordion-body border-radius">
              <ul className="sidebar-sub-menu">
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/ecommerce/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/ecommerce/" ? "active" : ""
                    }`}
                  >
                    eCommerce
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/crm"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/crm/" ? "active" : ""
                    }`}
                  >
                    CRM
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/project-management/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/project-management/"
                        ? "active"
                        : ""
                    }`}
                  >
                    Project Management
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/lms/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/lms/" ? "active" : ""
                    }`}
                  >
                    LMS
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/helpdesk/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/helpdesk/" ? "active" : ""
                    }`}
                  >
                    HelpDesk
                    <span className="trezo-badge">Hot</span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/analytics/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/analytics/" ? "active" : ""
                    }`}
                  >
                    Analytics
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/crypto/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/crypto/" ? "active" : ""
                    }`}
                  >
                    Crypto
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/sales/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/sales/" ? "active" : ""
                    }`}
                  >
                    Sales
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/hospital/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/hospital/" ? "active" : ""
                    }`}
                  >
                    Hospital
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/hrm/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/hrm/" ? "active" : ""
                    }`}
                  >
                    HRM
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/school/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/school/" ? "active" : ""
                    }`}
                  >
                    School
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/call-center/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/call-center/" ? "active" : ""
                    }`}
                  >
                    Call Center
                    <span className="trezo-badge style-two">Popular</span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/marketing/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/marketing/" ? "active" : ""
                    }`}
                  >
                    Marketing
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/nft/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/nft/" ? "active" : ""
                    }`}
                  >
                    NFT
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/saas/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/saas/" ? "active" : ""
                    }`}
                  >
                    SaaS
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/real-estate/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/real-estate/" ? "active" : ""
                    }`}
                  >
                    Real Estate
                    <span className="trezo-badge style-three">Top</span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/shipment/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/shipment/" ? "active" : ""
                    }`}
                  >
                    Shipment
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/finance/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/finance/" ? "active" : ""
                    }`}
                  >
                    Finance
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/pos-system/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/pos-system/" ? "active" : ""
                    }`}
                  >
                    POS System
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/podcast/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/podcast/" ? "active" : ""
                    }`}
                  >
                    Podcast
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/social-media/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/social-media-system/"
                        ? "active"
                        : ""
                    }`}
                  >
                    Social Media
                    <span className="trezo-badge">New</span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/doctor/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/doctor/" ? "active" : ""
                    }`}
                  >
                    Doctor
                    <span className="trezo-badge">New</span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/beauty-salon/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/beauty-salon/" ? "active" : ""
                    }`}
                  >
                    Beauty Salon
                    <span className="trezo-badge">New</span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/store-analytics/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/store-analytics/" ? "active" : ""
                    }`}
                  >
                    Store Analytics
                    <span className="trezo-badge">New</span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/dashboard/restaurant/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/dashboard/restaurant/" ? "active" : ""
                    }`}
                  >
                    Restaurant
                    <span className="trezo-badge">New</span>
                  </Link>
                </li>
              </ul>
            </div>
          </div>

          <div className="accordion-item border-radius border-0">
            <Button type="button" className="accordion-button">
              <i className="material-symbols-outlined">deployed_code</i>
              <span className="title" style={{ lineHeight: 1 }}>
                Apps
              </span>
            </Button>
            <div className="accordion-body border-radius">
              <ul className="sidebar-sub-menu">
                <li className="sidemenu-item">
                  <Link
                    href="/apps/to-do-list/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/apps/to-do-list/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">
                      format_list_bulleted
                    </i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      To Do List
                    </span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/apps/calendar/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/apps/calendar/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">date_range</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Calendar
                    </span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/apps/contacts/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/apps/contacts/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">contact_page</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Contacts
                    </span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/apps/chat/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/apps/chat/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">chat</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Chat
                    </span>
                  </Link>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">mail</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Email
                    </span>
                    <span className="trezo-badge style-two">3</span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/email/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/email/" ? "active" : ""
                          }`}
                        >
                          Inbox
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/email/compose/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/email/compose/" ? "active" : ""
                          }`}
                        >
                          Compose
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/email/read/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/email/read/" ? "active" : ""
                          }`}
                        >
                          Read
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Link
                    href="/apps/kanban-board/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/apps/kanban-board/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">team_dashboard</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Kanban Board
                    </span>
                  </Link>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">folder_open</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      File Manager
                    </span>
                    <span className="trezo-badge style-three">7</span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/file-manager/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/file-manager/" ? "active" : ""
                          }`}
                        >
                          My Drive
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/file-manager/assets/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/file-manager/assets/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Assets
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/file-manager/projects/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/file-manager/projects/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Projects
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/file-manager/personal/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/file-manager/personal/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Personal
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/file-manager/applications/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/file-manager/applications/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Applications
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/file-manager/documents/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/file-manager/documents/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Documents
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/apps/file-manager/media/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/apps/file-manager/media/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Media
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>
              </ul>
            </div>
          </div>

          <div className="accordion-item border-radius border-0">
            <Button type="button" className="accordion-button">
              <i className="material-symbols-outlined">layers</i>
              <span className="title" style={{ lineHeight: 1 }}>
                Pages
              </span>
            </Button>
            <div className="accordion-body border-radius">
              <ul className="sidebar-sub-menu">
                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">note_stack</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Front Pages
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/" ? "active" : ""
                          }`}
                        >
                          Home
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/front-pages/features/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/front-pages/features/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Features
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/front-pages/team/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/front-pages/team/" ? "active" : ""
                          }`}
                        >
                          Our Team
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/front-pages/faq/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/front-pages/faq/" ? "active" : ""
                          }`}
                        >
                          FAQ’s
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/front-pages/contact/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/front-pages/contact/" ? "active" : ""
                          }`}
                        >
                          Contact
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">shopping_cart</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      eCommerce
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Button type="button" className="accordion-button">
                          Products
                        </Button>
                        <div className="accordion-body border-radius">
                          <ul className="sidebar-sub-menu">
                            <li className="sidemenu-item">
                              <Link
                                href="/ecommerce/"
                                className={`sidemenu-link border-radius ${
                                  pathname === "/ecommerce/" ? "active" : ""
                                }`}
                              >
                                Products Grid
                              </Link>
                            </li>
                            <li className="sidemenu-item">
                              <Link
                                href="/ecommerce/products-list/"
                                className={`sidemenu-link border-radius ${
                                  pathname === "/ecommerce/products-list/"
                                    ? "active"
                                    : ""
                                }`}
                              >
                                Products List
                              </Link>
                            </li>
                            <li className="sidemenu-item">
                              <Link
                                href="/ecommerce/products-list/details/"
                                className={`sidemenu-link border-radius ${
                                  pathname ===
                                  "/ecommerce/products-list/details/"
                                    ? "active"
                                    : ""
                                }`}
                              >
                                Product Details
                              </Link>
                            </li>
                            <li className="sidemenu-item">
                              <Link
                                href="/ecommerce/create-product/"
                                className={`sidemenu-link border-radius ${
                                  pathname === "/ecommerce/create-product/"
                                    ? "active"
                                    : ""
                                }`}
                              >
                                Create Product
                              </Link>
                            </li>
                            <li className="sidemenu-item">
                              <Link
                                href="/ecommerce/edit-product/"
                                className={`sidemenu-link border-radius ${
                                  pathname === "/ecommerce/edit-product/"
                                    ? "active"
                                    : ""
                                }`}
                              >
                                Edit Product
                              </Link>
                            </li>
                          </ul>
                        </div>
                      </li>

                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/cart/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/cart/" ? "active" : ""
                          }`}
                        >
                          Cart
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/checkout/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/checkout/" ? "active" : ""
                          }`}
                        >
                          Checkout
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/orders/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/orders/" ? "active" : ""
                          }`}
                        >
                          Orders
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/orders/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/orders/details/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Order Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/orders/create/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/orders/create/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Create Order
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/orders/tracking/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/orders/tracking/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Order Tracking
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/customers/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/customers/" ? "active" : ""
                          }`}
                        >
                          Customers
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/customers/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/customers/details/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Customer Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/categories/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/categories/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Categories
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/sellers/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/sellers/" ? "active" : ""
                          }`}
                        >
                          Sellers
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/sellers/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/sellers/details/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Seller Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/sellers/create/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/sellers/create/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Create Seller
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/reviews/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/reviews/" ? "active" : ""
                          }`}
                        >
                          Reviews
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/ecommerce/refunds/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/ecommerce/refunds/" ? "active" : ""
                          }`}
                        >
                          Refunds
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">handshake</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      CRM
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/crm/contacts/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/crm/contacts/" ? "active" : ""
                          }`}
                        >
                          Contacts
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/crm/customers/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/crm/customers/" ? "active" : ""
                          }`}
                        >
                          Customers
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/crm/leads/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/crm/leads/" ? "active" : ""
                          }`}
                        >
                          Leads
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/crm/deals/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/crm/deals/" ? "active" : ""
                          }`}
                        >
                          Deals
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">description</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Project Management
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/project-management/project-overview/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/project-management/project-overview/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Project Overview
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/project-management/projects-list/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/project-management/projects-list/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Projects List
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/project-management/create-project/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/project-management/create-project/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Create Project
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/project-management/clients/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/project-management/clients/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Clients
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/project-management/teams/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/project-management/teams/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Teams
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/project-management/kanban-board/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/project-management/kanban-board/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Kanban Board
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/project-management/users/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/project-management/users/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Users
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">auto_stories</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      LMS
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/lms/courses/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/lms/courses/" ? "active" : ""
                          }`}
                        >
                          Courses List
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/lms/courses/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/lms/courses/details/" ? "active" : ""
                          }`}
                        >
                          Course Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/lms/lesson-preview/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/lms/lesson-preview/" ? "active" : ""
                          }`}
                        >
                          Lesson Preview
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/lms/create-course/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/lms/create-course/" ? "active" : ""
                          }`}
                        >
                          Create Course
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/lms/edit-course/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/lms/edit-course/" ? "active" : ""
                          }`}
                        >
                          Edit Course
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/lms/instructors/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/lms/instructors/" ? "active" : ""
                          }`}
                        >
                          Instructors
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">support</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      HelpDesk
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/helpdesk/tickets/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/helpdesk/tickets/" ? "active" : ""
                          }`}
                        >
                          Tickets
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/helpdesk/tickets/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/helpdesk/tickets/details/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Ticket Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/helpdesk/agents/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/helpdesk/agents/" ? "active" : ""
                          }`}
                        >
                          Agents
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/helpdesk/reports/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/helpdesk/reports/" ? "active" : ""
                          }`}
                        >
                          Reports
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">store</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      NFT Marketplace
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/nft/marketplace/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/nft/marketplace/" ? "active" : ""
                          }`}
                        >
                          Marketplace
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/nft/explore-all/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/nft/explore-all/" ? "active" : ""
                          }`}
                        >
                          Explore All
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/nft/live-auction/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/nft/live-auction/" ? "active" : ""
                          }`}
                        >
                          Live Auction
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/nft/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/nft/details/" ? "active" : ""
                          }`}
                        >
                          NFT Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/nft/creators/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/nft/creators/" ? "active" : ""
                          }`}
                        >
                          Creators
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/nft/creators/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/nft/creators/details/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Creator Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/nft/wallet-connect/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/nft/wallet-connect/" ? "active" : ""
                          }`}
                        >
                          Wallet Connect
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">
                      real_estate_agent
                    </i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Real Estate
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/real-estate/property-list/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/real-estate/property-list/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Property List
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/real-estate/property-details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/real-estate/property-details/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Property Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/real-estate/add-property/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/real-estate/add-property/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Add Property
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/real-estate/agents/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/real-estate/agents/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Agents
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/real-estate/agents/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/real-estate/agents/details/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Agent Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/real-estate/agents/add/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/real-estate/agents/add/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Add Agent
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/real-estate/customers/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/real-estate/customers/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Customers
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">calculate</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Finance
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/finance/wallet/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/finance/wallet/" ? "active" : ""
                          }`}
                        >
                          Wallet
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/finance/transactions/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/finance/transactions/" ? "active" : ""
                          }`}
                        >
                          Transactions
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">local_activity</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Events
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/events/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/events/" ? "active" : ""
                          }`}
                        >
                          Events Grid
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/events/list/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/events/list/" ? "active" : ""
                          }`}
                        >
                          Events List
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/events/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/events/details/" ? "active" : ""
                          }`}
                        >
                          Event Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/events/create-an-event/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/events/create-an-event/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Create An Event
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/events/edit-an-event/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/events/edit-an-event/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Edit An Event
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">share</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Social
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/social/profile/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/social/profile/" ? "active" : ""
                          }`}
                        >
                          Profile
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/social/settings/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/social/settings/" ? "active" : ""
                          }`}
                        >
                          Settings
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">content_paste</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Invoices
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/invoices/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/invoices/" ? "active" : ""
                          }`}
                        >
                          Invoices
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/invoices/details/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/invoices/details/" ? "active" : ""
                          }`}
                        >
                          Invoice Details
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/invoices/create/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/invoices/create/" ? "active" : ""
                          }`}
                        >
                          Create Invoice
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/invoices/edit/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/invoices/edit/" ? "active" : ""
                          }`}
                        >
                          Edit Invoice
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">person</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Users
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/users/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/users/" ? "active" : ""
                          }`}
                        >
                          Team Members
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/users/users-list/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/users/users-list/" ? "active" : ""
                          }`}
                        >
                          Users List
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/users/add-user/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/users/add-user/" ? "active" : ""
                          }`}
                        >
                          Add User
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">account_box</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Profile
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/profile/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/profile/" ? "active" : ""
                          }`}
                        >
                          User Profile
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/profile/teams/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/profile/teams/" ? "active" : ""
                          }`}
                        >
                          Teams
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/profile/projects/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/profile/projects/" ? "active" : ""
                          }`}
                        >
                          Projects
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Link
                    href="/starter/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/starter/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">star_border</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Starter
                    </span>
                  </Link>
                </li>
              </ul>
            </div>
          </div>

          <div className="accordion-item border-radius border-0">
            <Button type="button" className="accordion-button">
              <i className="material-symbols-outlined">token</i>
              <span className="title" style={{ lineHeight: 1 }}>
                Modules
              </span>
            </Button>
            <div className="accordion-body border-radius">
              <ul className="sidebar-sub-menu">
                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">qr_code_scanner</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      UI Elements
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/autocomplete/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/autocomplete/" ? "active" : ""
                          }`}
                        >
                          Autocomplete
                        </Link>
                      </li>

                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/buttons/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/buttons/" ? "active" : ""
                          }`}
                        >
                          Buttons
                        </Link>
                      </li>

                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/checkbox/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/checkbox/" ? "active" : ""
                          }`}
                        >
                          Checkbox
                        </Link>
                      </li>

                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/radio/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/radio/" ? "active" : ""
                          }`}
                        >
                          Radio
                        </Link>
                      </li>

                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/rating/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/rating/" ? "active" : ""
                          }`}
                        >
                          Rating
                        </Link>
                      </li>

                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/select/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/select/" ? "active" : ""
                          }`}
                        >
                          Select
                        </Link>
                      </li>

                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/slider/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/slider/" ? "active" : ""
                          }`}
                        >
                          Slider
                        </Link>
                      </li> 

                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/avatar/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/avatar/" ? "active" : ""
                          }`}
                        >
                          Avatar
                        </Link>
                      </li>

                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/badge/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/badge/" ? "active" : ""
                          }`}
                        >
                          Badge
                        </Link>
                      </li>
      
                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/alerts/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/alerts/" ? "active" : ""
                          }`}
                        >
                          Alerts
                        </Link>
                      </li>
 
                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/progress/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/progress/" ? "active" : ""
                          }`}
                        >
                          Progress
                        </Link>
                      </li>
 
                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/accordion/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/accordion/" ? "active" : ""
                          }`}
                        >
                          Accordion
                        </Link>
                      </li>

                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/card/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/card/" ? "active" : ""
                          }`}
                        >
                          Card
                        </Link>
                      </li>
     
                      <li className="sidemenu-item">
                        <Link
                          href="/ui-kit/tabs/"
                          className={`sidemenu-link ${
                            pathname === "/ui-kit/tabs/" ? "active" : ""
                          }`}
                        >
                          Tabs
                        </Link>
                      </li> 
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">emoji_emotions</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Icons
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/icons/material-symbols/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/icons/material-symbols/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Material Symbols
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/icons/remixicon/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/icons/remixicon/" ? "active" : ""
                          }`}
                        >
                          RemixIcon
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Link
                    href="/tables/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/tables/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">table_chart</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Tables
                    </span>
                  </Link>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">lock_open</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Authentication
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/authentication/sign-in/"
                          className="sidemenu-link border-radius"
                        >
                          Sign In
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/authentication/sign-up/"
                          className="sidemenu-link border-radius"
                        >
                          Sign Up
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/authentication/forgot-password/"
                          className="sidemenu-link border-radius"
                        >
                          Forgot Password
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/authentication/reset-password/"
                          className="sidemenu-link border-radius"
                        >
                          Reset Password
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/authentication/confirm-email/"
                          className="sidemenu-link border-radius"
                        >
                          Confirm Email
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/authentication/lock-screen/"
                          className="sidemenu-link border-radius"
                        >
                          Lock Screen
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/authentication/logout/"
                          className="sidemenu-link border-radius"
                        >
                          Logout
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">content_copy</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Extra Pages
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/pricing/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/pricing/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Pricing
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/timeline/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/timeline/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Timeline
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/faq/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/faq/" ? "active" : ""
                          }`}
                        >
                          FAQ
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/gallery/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/gallery/" ? "active" : ""
                          }`}
                        >
                          Gallery
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/testimonials/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/testimonials/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Testimonials
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/search/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/search/" ? "active" : ""
                          }`}
                        >
                          Search
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/blank-page/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/blank-page/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Blank Page
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">error</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Errors
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/not-found/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/not-found/" ? "active" : ""
                          }`}
                        >
                          404 Error Page
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/internal-error/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/internal-error/" ? "active" : ""
                          }`}
                        >
                          Internal Error
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Link
                    href="/widgets/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/widgets/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">widgets</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Widgets
                    </span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/maps/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/maps/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">map</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Maps
                    </span>
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/notifications/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/notifications/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">notifications</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Notifications
                    </span>
                  </Link>
                </li>

                <li className="sidemenu-item">
                  <Link
                    href="/members/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/members/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">people</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Members
                    </span>
                  </Link>
                </li>
              </ul>
            </div>
          </div>

          <div className="accordion-item border-radius border-0">
            <Button type="button" className="accordion-button">
              <i className="material-symbols-outlined">forum</i>
              <span className="title" style={{ lineHeight: 1 }}>
                Forms
              </span>
            </Button>
            <div className="accordion-body border-radius">
              <ul className="sidebar-sub-menu">
                <li className="sidemenu-item">
                  <Link
                    href="/forms/basic-elements/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/forms/basic-elements/" ? "active" : ""
                    }`}
                  >
                    Basic Elements
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/forms/advanced-elements/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/forms/advanced-elements/" ? "active" : ""
                    }`}
                  >
                    Advanced Elements
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/forms/editors/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/forms/editors/" ? "active" : ""
                    }`}
                  >
                    Editors
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/forms/file-uploader/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/forms/file-uploader/" ? "active" : ""
                    }`}
                  >
                    File Upload
                  </Link>
                </li>
              </ul>
            </div>
          </div>

          <div className="accordion-item border-radius border-0">
            <Button type="button" className="accordion-button">
              <i className="material-symbols-outlined">pie_chart</i>
              <span className="title" style={{ lineHeight: 1 }}>
                Charts
              </span>
            </Button>
            <div className="accordion-body border-radius">
              <ul className="sidebar-sub-menu">
                <li className="sidemenu-item">
                  <Link
                    href="/charts/line/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/charts/line/" ? "active" : ""
                    }`}
                  >
                    Line
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/charts/area/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/charts/area/" ? "active" : ""
                    }`}
                  >
                    Area
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/charts/column/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/charts/column/" ? "active" : ""
                    }`}
                  >
                    Column
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/charts/mixed/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/charts/mixed/" ? "active" : ""
                    }`}
                  >
                    Mixed
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/charts/radialbar/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/charts/radialbar/" ? "active" : ""
                    }`}
                  >
                    RadialBar
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/charts/radar/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/charts/radar/" ? "active" : ""
                    }`}
                  >
                    Radar
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/charts/pie/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/charts/pie/" ? "active" : ""
                    }`}
                  >
                    Pie
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/charts/polar/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/charts/polar/" ? "active" : ""
                    }`}
                  >
                    Polar
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/charts/more/"
                    className={`sidemenu-link border-radius ${
                      pathname === "/charts/more/" ? "active" : ""
                    }`}
                  >
                    More
                  </Link>
                </li>
              </ul>
            </div>
          </div>

          <div className="accordion-item border-radius border-0">
            <Button type="button" className="accordion-button">
              <i className="material-symbols-outlined">open_run</i>
              <span className="title" style={{ lineHeight: 1 }}>
                Others
              </span>
            </Button>
            <div className="accordion-body border-radius">
              <ul className="sidebar-sub-menu">
                <li className="sidemenu-item">
                  <Link
                    href="/my-profile/"
                    className={`sidemenu-link with-icon border-radius ${
                      pathname === "/my-profile/" ? "active" : ""
                    }`}
                  >
                    <i className="material-symbols-outlined">account_circle</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      My Profile
                    </span>
                  </Link>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">settings</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Settings
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link
                          href="/settings/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/settings/" ? "active" : ""
                          }`}
                        >
                          Account Settings
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/settings/change-password/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/settings/change-password/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Change Password
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/settings/connections/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/settings/connections/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Connections
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/settings/privacy-policy/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/settings/privacy-policy/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Privacy Policy
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link
                          href="/settings/terms-conditions/"
                          className={`sidemenu-link border-radius ${
                            pathname === "/settings/terms-conditions/"
                              ? "active"
                              : ""
                          }`}
                        >
                          Terms & Conditions
                        </Link>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Button
                    type="button"
                    className="accordion-button with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">unfold_more</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Multi Level Menu
                    </span>
                  </Button>
                  <div className="accordion-body border-radius">
                    <ul className="sidebar-sub-menu">
                      <li className="sidemenu-item">
                        <Link href="#" className="sidemenu-link border-radius">
                          First
                        </Link>
                      </li>
                      <li className="sidemenu-item">
                        <Link href="#" className="sidemenu-link border-radius">
                          Third
                        </Link>
                      </li>

                      <li className="sidemenu-item">
                        <Button type="button" className="accordion-button">
                          Third
                          <span className="trezo-badge">3</span>
                        </Button>
                        <div className="accordion-body border-radius">
                          <ul className="sidebar-sub-menu">
                            <li className="sidemenu-item">
                              <Link
                                href="#"
                                className="sidemenu-link border-radius"
                              >
                                Third 1
                              </Link>
                            </li>

                            <li className="sidemenu-item">
                              <Button
                                type="button"
                                className="accordion-button"
                              >
                                Third 2
                              </Button>
                              <div className="accordion-body border-radius">
                                <ul className="sidebar-sub-menu">
                                  <li className="sidemenu-item">
                                    <Link
                                      href="#"
                                      className="sidemenu-link border-radius"
                                    >
                                      Four 1
                                    </Link>
                                  </li>
                                </ul>
                              </div>
                            </li>
                            <li className="sidemenu-item">
                              <Link
                                href="#"
                                className="sidemenu-link border-radius"
                              >
                                Third 3
                              </Link>
                            </li>
                          </ul>
                        </div>
                      </li>
                    </ul>
                  </div>
                </li>

                <li className="sidemenu-item">
                  <Link
                    href="/authentication/logout"
                    className="sidemenu-link with-icon border-radius"
                  >
                    <i className="material-symbols-outlined">logout</i>
                    <span className="title" style={{ lineHeight: 1 }}>
                      Logout
                    </span>
                  </Link>
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </>
  );
};

export default HorizontalNavbar;
