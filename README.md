# Audiody

**Audiody** is an audiobook and podcast listening application, prioritizing audiobooks while offering a seamless experience for podcast enthusiasts. This project is in its early stages, but it aims to become a go-to app for audiobook and podcast lovers.

## Usage  (not true for this project yet)
To run the app, you currently need to create an API Key yourself. Don't worry, the Drive API is completely free. For that, you need a Google account. If you have that on hand, go to the Google Cloud Console and create or select a project.
After that, select the Google Drive API in the API Library and click "Activate". As soon as it's active, you can visit the Drive API Overview where you can select "Credentials" and then create a new set of credentials.
For the credentials type, you have to select "OAuth Client ID" and the application type will be "Desktop App". You can choose the name as you want. Now that you have valid credentials for the API, please click the Download button to the right to download the client credentials file. You need to place this as clientCredentials.json in the directory where you have your local copy of gdrivefs-rust.

## Goals and Features

- **Audiobook and Podcast Integration**
  - List and play audiobooks and podcasts from YouTube.
  - Start with integration of Librivox for free audiobooks.
  - Enable connection to Google Drive for user-uploaded audiobooks and podcasts.

- **User Experience**
  - Focus on **great design** inspired by [Inner Tune](https://github.com/z-huang/InnerTune) and [Shortwave](https://apps.gnome.org/en-GB/Shortwave/).
  - Provide **chapter timestamps** for quick navigation.
  - Implement **book tracking** for remembering progress across multiple audiobooks and podcasts.

- **Additional Features**
  - Offer **transcripts** for audiobooks and podcasts (where available).
  - Include **chapter timestamps** for enhanced navigation.

## Current Inspiration

- [Inner Tune](https://github.com/TimonT/SoundcloudDesktop)
- [Shortwave](https://apps.gnome.org/en-GB/Shortwave/)

## Questions for Improvement

1. **Design:** What specific elements from "Inner Tune" or "Shortwave" should we replicate or improve upon?
2. **Audiobook Sources:** Besides Librivox and Google Drive, are there other sources we should integrate (e.g., Audible, Project Gutenberg)?
3. **User Experience:** What features would make navigation and playback more intuitive for users?
4. **Transcripts:** Should we explore automated transcript generation for podcasts or limit this feature to existing transcripts?
5. **Community Input:** How can we involve the community in creating features like chapter timestamps or book summaries?
6. **Offline Support:** Would offline listening for Google Drive content or Librivox audiobooks be a priority?

---

## Development Roadmap

### Phase 1: Core Features
- Integrate Librivox API for audiobooks.
- Create a basic UI to list, search, and play audiobooks.
- Add Google Drive connectivity for user-uploaded content.

### Phase 2: Enhanced Features
- Implement chapter timestamps and book tracking.
- Add YouTube podcast support.
- Design a user-friendly interface inspired by Inner Tune and Shortwave.

### Phase 3: Further Features
- Offer transcripts for audiobooks and podcasts.
- Provide advanced filtering options (e.g., genre, duration).

---

## Contributing

We welcome contributions! Here’s how you can get involved:
- Submit feedback on design and functionality via the Issues tab.
- Contribute to feature development by forking the repository and submitting pull requests.
- Share your ideas on additional features or integrations.

## License

This project is licensed under the **MIT License**.

---

Let’s build a powerful and user-friendly audiobook and podcast app together!
