# Offline playback integration with synthetic audio and a self-signed local TLS fixture.
# Only the test source disables certificate validation; application TLS is unchanged.
import os,subprocess,tempfile,pathlib,http.server,threading,functools,sys,ssl
app=pathlib.Path(sys.argv[1]).resolve(); lib=app/'usr/lib'
with tempfile.TemporaryDirectory() as d:
 p=pathlib.Path(d)
 env={**os.environ,'LD_LIBRARY_PATH':str(lib),'GST_PLUGIN_SYSTEM_PATH_1_0':str(lib/'gstreamer-1.0'),'GST_PLUGIN_PATH_1_0':str(lib/'gstreamer-1.0'),'GST_PLUGIN_SCANNER_1_0':str(lib/'gstreamer1.0/gstreamer-1.0/gst-plugin-scanner'),'GST_REGISTRY':str(p/'registry.bin'),'GIO_MODULE_DIR':str(lib/'gio/modules')}
 env.pop('PYTHONHOME',None);env.pop('PYTHONPATH',None)
 for element in ['fakevideosink','webvttenc','scaletempo','appsink']:
  subprocess.run(['gst-inspect-1.0',element],env=env,check=True,stdout=subprocess.DEVNULL)
 print('WebKit media factories available')
 subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-keyout',str(p/'key.pem'),'-out',str(p/'cert.pem'),'-days','1','-subj','/CN=localhost','-addext','subjectAltName=IP:127.0.0.1'],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 env['SSL_CERT_FILE']=str(p/'cert.pem')
 server=http.server.ThreadingHTTPServer(('127.0.0.1',0),functools.partial(http.server.SimpleHTTPRequestHandler,directory=d)); threading.Thread(target=server.serve_forever,daemon=True).start()
 tls=http.server.ThreadingHTTPServer(('127.0.0.1',0),functools.partial(http.server.SimpleHTTPRequestHandler,directory=d))
 context=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);context.load_cert_chain(p/'cert.pem',p/'key.pem');tls.socket=context.wrap_socket(tls.socket,server_side=True)
 threading.Thread(target=tls.serve_forever,daemon=True).start()
 try:
  for ext,codec in [('mp3','libmp3lame'),('m4a','aac'),('webm','libopus'),('ogg','libvorbis'),('flac','flac')]:
   sample=p/('audio.'+ext)
   subprocess.run(['/usr/bin/ffmpeg','-v','error','-f','lavfi','-i','sine=frequency=440:duration=0.3','-c:a',codec,str(sample)],check=True)
   for uri in [sample.as_uri(),f'http://127.0.0.1:{server.server_port}/{sample.name}',f'https://127.0.0.1:{tls.server_port}/{sample.name}']:
    pipeline=['playbin',f'uri={uri}','audio-sink=fakesink','video-sink=fakesink']
    if uri.startswith('https:'):
     pipeline=['souphttpsrc',f'location={uri}','ssl-strict=false','!','decodebin','!','audioconvert','!','audioresample','!','fakesink']
    r=subprocess.run(['gst-launch-1.0','-q',*pipeline],env=env,capture_output=True,text=True,timeout=15)
    if r.returncode:raise RuntimeError(r.stderr)
   print(ext,'file + HTTP + HTTPS playback passed')
 finally:server.shutdown();tls.shutdown()
