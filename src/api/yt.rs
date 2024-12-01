use std::vec;

use crate::api::types::*;
use serde_json::json;
use ureq::{Agent, Request, Response};
use scraper::{Html, Selector};

pub struct YouTubeClient {
    client: reqwest::Client,
    base_url: String,
}

impl YouTubeClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            // TODO: make this configurable and automaticaly detect best  url
            // https://pipedapi.kavin.rocks/opensearch/suggestions?query=Audiobook
            base_url: "https://www.youtube.com/".to_string(),
        }
    }
}

// https://librivox.app/search.jsp?search=marxism

impl YouTubeClient {
  pub fn search(&self, query: String) -> Result<Vec<Book>, ureq::Error> {
    let url = format!("{}results?search_query={}", self.base_url, query.replace(" ", "+"));
    let body: String = ureq::get(&url)
      .call()?
      .into_string()?;

    log::info!("Youtube search: {}", url);

    log::info!("Body: {}", body);
    
    let document = Html::parse_document(&body);
    let video_selector = Selector::parse("ytd-video-renderer").unwrap();

    Ok(document.select(&video_selector)
    .map(|book_element| {
      let title_selector = Selector::parse("#video-title").unwrap();
      let title_element = book_element.select(&title_selector).next();
      let title = title_element
      .map(|e| e.text().collect::<String>().trim().to_string())
      .unwrap_or_default();

      log::info!("Title: {}", title);

      let book_url = book_element.value().attr("href").unwrap_or("").to_string();
      
      let author_selector = Selector::parse("#channel-name a").unwrap();
      let author = book_element
          .select(&author_selector)
          .next()
          .map(|e| e.text().collect::<String>().trim().to_string())
          .unwrap_or_default();

      let cover_selector = Selector::parse("span.ytd-thumbnail-overlay-time-status-renderer").unwrap();
      let cover_url = book_element
      .select(&cover_selector)
      .next()
      .and_then(|img| img.value().attr("src"))
      .map(|src| {
          if src.starts_with("http") {
              src.to_string()
          } else {
              format!("{}{}", self.base_url.trim_end_matches('/'), src)
          }
      })
      .unwrap_or("".to_string());
    
      Book {
        saved: false,
        title,
        description: "".to_string(),
        author,
        url: book_url,
        image_URL: cover_url,
        chapter_urls: vec![],
        chapter_durations: vec![],
        chapter_reader: vec![],
      }
    }).collect())
  }
}
// 

// 

//

//

//

/*
<ytd-video-renderer class="style-scope ytd-item-section-renderer" bigger-thumbs-style="BIG" lockup="true" is-search="" use-search-ui="" use-bigger-thumbs="" inline-title-icon=""><!--css-build:shady--><!--css-build:shady--><div id="dismissible" class="style-scope ytd-video-renderer">
  <ytd-thumbnail use-hovered-property="" class="style-scope ytd-video-renderer" size="large" loaded=""><!--css-build:shady--><!--css-build:shady--><a id="thumbnail" class="yt-simple-endpoint inline-block style-scope ytd-thumbnail" aria-hidden="true" tabindex="-1" rel="null" href="/watch?v=A4TU2h_rDlM&amp;pp=ygUJYXVkaW9ib29r">
  <yt-image alt="" ftl-eligible="" notify-on-loaded="" notify-on-unloaded="" class="style-scope ytd-thumbnail"><img alt="" style="background-color: transparent;" class="yt-core-image yt-core-image--fill-parent-height yt-core-image--fill-parent-width yt-core-image--content-mode-scale-aspect-fill yt-core-image--loaded" src="https://i.ytimg.com/vi/A4TU2h_rDlM/hq720.jpg?sqp=-oaymwE2CNAFEJQDSFXyq4qpAygIARUAAIhCGAFwAcABBvABAfgB_gmAAtAFigIMCAAQARhlIFwoSTAP&amp;rs=AOn4CLAfxp6thQKWSJCBSRmrdidTcp6OZA"></yt-image>
  
  <div id="overlays" class="style-scope ytd-thumbnail"><ytd-thumbnail-overlay-time-status-renderer class="style-scope ytd-thumbnail" hide-time-status="" overlay-style="DEFAULT"><!--css-build:shady--><!--css-build:shady--><ytd-badge-supported-renderer is-thumbnail-badge="" class="style-scope ytd-thumbnail-overlay-time-status-renderer" system-icons="" enable-refresh-web="" enable-signature-moments-web=""><!--css-build:shady--><!--css-build:shady--><dom-repeat id="repeat" as="badge" class="style-scope ytd-badge-supported-renderer"><template is="dom-repeat"></template></dom-repeat></ytd-badge-supported-renderer><div class="thumbnail-overlay-badge-shape style-scope ytd-thumbnail-overlay-time-status-renderer"><badge-shape class="badge-shape-wiz badge-shape-wiz--thumbnail-default badge-shape-wiz--thumbnail-badge" role="img" aria-label="65 hours, 14 minutes, 28 seconds"><div class="badge-shape-wiz__text">65:14:28</div></badge-shape></div><div id="time-status" class="style-scope ytd-thumbnail-overlay-time-status-renderer" hidden=""><yt-icon size="16" class="style-scope ytd-thumbnail-overlay-time-status-renderer" disable-upgrade="" hidden=""></yt-icon><span id="text" class="style-scope ytd-thumbnail-overlay-time-status-renderer" aria-label="65 hours, 14 minutes, 28 seconds">
    65:14:28
  </span></div></ytd-thumbnail-overlay-time-status-renderer><ytd-thumbnail-overlay-now-playing-renderer class="style-scope ytd-thumbnail" now-playing-badge=""><!--css-build:shady--><!--css-build:shady--><span id="overlay-text" class="style-scope ytd-thumbnail-overlay-now-playing-renderer">Now playing</span>
<ytd-thumbnail-overlay-equalizer class="style-scope ytd-thumbnail-overlay-now-playing-renderer"><!--css-build:shady--><!--css-build:shady--><svg xmlns="http://www.w3.org/2000/svg" id="equalizer" viewBox="0 0 55 95" class="style-scope ytd-thumbnail-overlay-equalizer">
  <g class="style-scope ytd-thumbnail-overlay-equalizer">
    <rect class="bar style-scope ytd-thumbnail-overlay-equalizer" x="0"></rect>
    <rect class="bar style-scope ytd-thumbnail-overlay-equalizer" x="20"></rect>
    <rect class="bar style-scope ytd-thumbnail-overlay-equalizer" x="40"></rect>
  </g>
</svg>
</ytd-thumbnail-overlay-equalizer>
</ytd-thumbnail-overlay-now-playing-renderer></div>
  <div id="mouseover-overlay" class="style-scope ytd-thumbnail"></div>
  <div id="hover-overlays" class="style-scope ytd-thumbnail"></div>
</a>
</ytd-thumbnail>
  <div class="text-wrapper style-scope ytd-video-renderer">
    <div id="meta" class="style-scope ytd-video-renderer">
      <div id="title-wrapper" class="style-scope ytd-video-renderer">
        <h3 class="title-and-badge style-scope ytd-video-renderer">
          <ytd-badge-supported-renderer collection-truncate="" class="style-scope ytd-video-renderer" disable-upgrade="" hidden="">
          </ytd-badge-supported-renderer>
          <a id="video-title" class="yt-simple-endpoint style-scope ytd-video-renderer" title="Complete Sherlock Holmes Audiobook Collection: All Novels &amp; Stories | Audiobook 🎧📚" aria-label="Complete Sherlock Holmes Audiobook Collection: All Novels &amp; Stories | Audiobook 🎧📚 by Gates of Imagination 516,809 views 3 months ago 65 hours" href="/watch?v=A4TU2h_rDlM&amp;pp=ygUJYXVkaW9ib29r">
            <yt-icon id="inline-title-icon" class="style-scope ytd-video-renderer" hidden=""><!--css-build:shady--><!--css-build:shady--></yt-icon>
            <yt-formatted-string class="style-scope ytd-video-renderer" aria-label="Complete Sherlock Holmes Audiobook Collection: All Novels &amp; Stories | Audiobook 🎧📚 by Gates of Imagination 516,809 views 3 months ago 65 hours">Complete Sherlock Holmes Audiobook Collection: All Novels &amp; Stories | Audiobook 🎧📚</yt-formatted-string>
          </a>
        </h3>
        <div id="menu" class="style-scope ytd-video-renderer"><ytd-menu-renderer class="style-scope ytd-video-renderer" safe-area="" menu-active=""><!--css-build:shady--><!--css_build_scope:ytd-menu-renderer--><!--css_build_styles:video.youtube.src.web.polymer.shared.ui.styles.yt_base_styles.yt.base.styles.css.js--><div id="top-level-buttons-computed" class="top-level-buttons style-scope ytd-menu-renderer"></div><div id="flexible-item-buttons" class="style-scope ytd-menu-renderer"></div><yt-icon-button id="button" class="dropdown-trigger style-scope ytd-menu-renderer" style-target="button" role="button" aria-label="yt-icon-button"><!--css-build:shady--><!--css-build:shady--><button id="button" class="style-scope yt-icon-button" aria-label="Action menu"><yt-icon class="style-scope ytd-menu-renderer"><!--css-build:shady--><!--css-build:shady--><span class="yt-icon-shape style-scope yt-icon yt-spec-icon-shape"><div style="width: 100%; height: 100%; display: block; fill: currentcolor;"><svg xmlns="http://www.w3.org/2000/svg" enable-background="new 0 0 24 24" height="24" viewBox="0 0 24 24" width="24" focusable="false" aria-hidden="true" style="pointer-events: none; display: inherit; width: 100%; height: 100%;"><path d="M12 16.5c.83 0 1.5.67 1.5 1.5s-.67 1.5-1.5 1.5-1.5-.67-1.5-1.5.67-1.5 1.5-1.5zM10.5 12c0 .83.67 1.5 1.5 1.5s1.5-.67 1.5-1.5-.67-1.5-1.5-1.5-1.5.67-1.5 1.5zm0-6c0 .83.67 1.5 1.5 1.5s1.5-.67 1.5-1.5-.67-1.5-1.5-1.5-1.5.67-1.5 1.5z"></path></svg></div></span></yt-icon></button><yt-interaction id="interaction" class="circular style-scope yt-icon-button"><!--css-build:shady--><!--css-build:shady--><div class="stroke style-scope yt-interaction"></div><div class="fill style-scope yt-interaction"></div></yt-interaction></yt-icon-button><yt-button-shape id="button-shape" version="modern" class="style-scope ytd-menu-renderer" disable-upgrade="" hidden=""></yt-button-shape></ytd-menu-renderer></div>
      </div>
      <ytd-video-meta-block class="style-scope ytd-video-renderer byline-separated" amsterdam-post-mvp=""><!--css-build:shady--><!--css-build:shady-->
<div id="metadata" class="style-scope ytd-video-meta-block">
  <div id="byline-container" class="style-scope ytd-video-meta-block" hidden="">
    <ytd-channel-name id="channel-name" class=" style-scope ytd-video-meta-block style-scope ytd-video-meta-block"><!--css-build:shady--><!--css-build:shady--><div id="container" class="style-scope ytd-channel-name">
  <div id="text-container" class="style-scope ytd-channel-name">
    <yt-formatted-string id="text" link-inherit-color="" respect-lang-dir="" title="Gates of Imagination" class="style-scope ytd-channel-name complex-string" ellipsis-truncate="" ellipsis-truncate-styling="" dir="auto" style="text-align: left;" has-link-only_=""><a class="yt-simple-endpoint style-scope yt-formatted-string" spellcheck="false" href="/@gatesofimagination">Gates of Imagination</a></yt-formatted-string>
  </div>
  <tp-yt-paper-tooltip fit-to-visible-bounds="" class="style-scope ytd-channel-name" role="tooltip" tabindex="-1" aria-label="tooltip"><!--css-build:shady--><div id="tooltip" class="hidden style-scope tp-yt-paper-tooltip" style-target="tooltip">
  
    Gates of Imagination
  
</div>
</tp-yt-paper-tooltip>
</div>
<ytd-badge-supported-renderer class="style-scope ytd-channel-name" disable-upgrade="" hidden="">
</ytd-badge-supported-renderer>
</ytd-channel-name>
    <div id="separator" class="style-scope ytd-video-meta-block">•</div>
    <yt-formatted-string id="video-info" class="style-scope ytd-video-meta-block" is-empty="function(){var e=Da.apply(0,arguments);a.loggingStatus.currentExternalCall=b;a.loggingStatus.bypassProxyController=!0;var g,k=((g=a.is)!=null?g:a.tagName).toLowerCase();Uy(k,b,&quot;PROPERTY_ACCESS_CALL_EXTERNAL&quot;);var m;g=(m=c!=null?c:d[b])==null?void 0:m.call.apply(m,[d].concat(ma(e)));a.loggingStatus.currentExternalCall=void 0;a.loggingStatus.bypassProxyController=!1;return g}" hidden=""><!--css-build:shady--><!--css-build:shady--><yt-attributed-string class="style-scope yt-formatted-string"></yt-attributed-string></yt-formatted-string>
  </div>
  <div id="metadata-line" class="style-scope ytd-video-meta-block">
    
    <ytd-badge-supported-renderer class="inline-metadata-badge style-scope ytd-video-meta-block" hidden="" system-icons="" enable-refresh-web="" enable-signature-moments-web=""><!--css-build:shady--><!--css-build:shady--><dom-repeat id="repeat" as="badge" class="style-scope ytd-badge-supported-renderer"><template is="dom-repeat"></template></dom-repeat></ytd-badge-supported-renderer>
    <div id="separator" class="style-scope ytd-video-meta-block" hidden="">•</div>
    
      <span class="inline-metadata-item style-scope ytd-video-meta-block">516K views</span>
    
      <span class="inline-metadata-item style-scope ytd-video-meta-block">3 months ago</span>
    <dom-repeat strip-whitespace="" class="style-scope ytd-video-meta-block"><template is="dom-repeat"></template></dom-repeat>
  </div>
</div>
<div id="additional-metadata-line" class="style-scope ytd-video-meta-block">
  <dom-repeat class="style-scope ytd-video-meta-block"><template is="dom-repeat"></template></dom-repeat>
</div>

</ytd-video-meta-block>
    </div>
    <div id="channel-info" class="style-scope ytd-video-renderer">
      <a id="channel-thumbnail" class="style-scope ytd-video-renderer" href="/@gatesofimagination" aria-label="Go to channel">
        <yt-img-shadow width="24" class="style-scope ytd-video-renderer no-transition" style="background-color: transparent;" loaded=""><!--css-build:shady--><!--css-build:shady--><img id="img" draggable="false" class="style-scope yt-img-shadow" alt="" width="24" src="https://yt3.ggpht.com/I2L81gEUrj8Z0aRo5TYIvB2OqsnlNkzNplSFD9h_IkTNszvRGG9rUZBNrK1mO47vtm-P-tU5=s68-c-k-c0x00ffffff-no-rj"></yt-img-shadow>
      </a>
      <ytd-channel-name id="channel-name" class="long-byline style-scope ytd-video-renderer" wrap-text="true"><!--css-build:shady--><!--css-build:shady--><div id="container" class="style-scope ytd-channel-name">
  <div id="text-container" class="style-scope ytd-channel-name">
    <yt-formatted-string id="text" link-inherit-color="" respect-lang-dir="" title="" class="style-scope ytd-channel-name" dir="auto" style="text-align: left;" has-link-only_=""><a class="yt-simple-endpoint style-scope yt-formatted-string" spellcheck="false" href="/@gatesofimagination">Gates of Imagination</a></yt-formatted-string>
  </div>
  <tp-yt-paper-tooltip fit-to-visible-bounds="" class="style-scope ytd-channel-name" role="tooltip" tabindex="-1" aria-label="tooltip"><!--css-build:shady--><div id="tooltip" class="hidden style-scope tp-yt-paper-tooltip" style-target="tooltip">
  
    Gates of Imagination
  
</div>
</tp-yt-paper-tooltip>
</div>
<ytd-badge-supported-renderer class="style-scope ytd-channel-name" disable-upgrade="" hidden="">
</ytd-badge-supported-renderer>
</ytd-channel-name>
    </div>
    <yt-formatted-string id="description-text" class="style-scope ytd-video-renderer" is-empty="function(){var e=Da.apply(0,arguments);a.loggingStatus.currentExternalCall=b;a.loggingStatus.bypassProxyController=!0;var g,k=((g=a.is)!=null?g:a.tagName).toLowerCase();Uy(k,b,&quot;PROPERTY_ACCESS_CALL_EXTERNAL&quot;);var m;g=(m=c!=null?c:d[b])==null?void 0:m.call.apply(m,[d].concat(ma(e)));a.loggingStatus.currentExternalCall=void 0;a.loggingStatus.bypassProxyController=!1;return g}" hidden=""><!--css-build:shady--><!--css-build:shady--><yt-attributed-string class="style-scope yt-formatted-string"></yt-attributed-string></yt-formatted-string>
    
      <div class="metadata-snippet-container style-scope ytd-video-renderer style-scope ytd-video-renderer">
        <a class="yt-simple-endpoint metadata-snippet-timestamp style-scope ytd-video-renderer" hidden="">
          <span id="time" class="style-scope ytd-video-renderer"></span>
          <yt-formatted-string class="metadata-snippet-text-navigation style-scope ytd-video-renderer"><span dir="auto" class="style-scope yt-formatted-string">The </span><span dir="auto" class="bold style-scope yt-formatted-string" style-target="bold">audiobook</span><span dir="auto" class="style-scope yt-formatted-string"> collection "Sherlock Holmes. The Ultimate Collection" is a true treasure for fans of classic detective literature.</span></yt-formatted-string>
        </a>
        <yt-formatted-string class="metadata-snippet-text style-scope ytd-video-renderer"><span dir="auto" class="style-scope yt-formatted-string">The </span><span dir="auto" class="bold style-scope yt-formatted-string" style-target="bold">audiobook</span><span dir="auto" class="style-scope yt-formatted-string"> collection "Sherlock Holmes. The Ultimate Collection" is a true treasure for fans of classic detective literature.</span></yt-formatted-string>
        <tp-yt-paper-tooltip class="style-scope ytd-video-renderer" role="tooltip" tabindex="-1" aria-label="tooltip"><!--css-build:shady--><div id="tooltip" class="hidden style-scope tp-yt-paper-tooltip" style-target="tooltip">
  From the video description
</div>
</tp-yt-paper-tooltip>
      </div>
    <dom-repeat class="style-scope ytd-video-renderer"><template is="dom-repeat"></template></dom-repeat>
    <ytd-badge-supported-renderer id="badges" class="style-scope ytd-video-renderer" disable-upgrade="" hidden="">
    </ytd-badge-supported-renderer>
    <div id="expandable-metadata" class="style-scope ytd-video-renderer"></div>
    <div id="buttons" class="style-scope ytd-video-renderer"></div>
  </div>
</div>
<div id="dismissed" class="style-scope ytd-video-renderer"></div>
<yt-interaction id="interaction" class="extended style-scope ytd-video-renderer"><!--css-build:shady--><!--css-build:shady--><div class="stroke style-scope yt-interaction"></div><div class="fill style-scope yt-interaction"></div></yt-interaction>
</ytd-video-renderer>
*/