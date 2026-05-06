import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { HomePage, CreatePage, ChannelPage, ChannelListPage, NotFoundPage } from '@/pages';
import { ScrollToTop, PageTransition, ErrorBoundary, WechatProvider } from '@/components';

function App() {
  return (
    <BrowserRouter>
      <ScrollToTop />
      <WechatProvider>
        <ErrorBoundary>
          <PageTransition>
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route path="/create" element={<CreatePage />} />
              <Route path="/channels" element={<ChannelListPage />} />
              <Route path="/channel/:channelId" element={<ChannelPage />} />
              <Route path="*" element={<NotFoundPage />} />
            </Routes>
          </PageTransition>
        </ErrorBoundary>
      </WechatProvider>
    </BrowserRouter>
  );
}

export default App;
