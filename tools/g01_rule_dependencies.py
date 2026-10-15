"""Direct rule dependencies of the accepted registration allocation (not closure)."""
def sem(*ns):return ["AF-SEMANTIC-%04d"%n for n in ns]
def contract(*ns):return ["AF-CONTRACT-%04d"%n for n in ns]
DEPENDENCIES={
197:contract(17)+sem(142),198:sem(160,161,182,187),199:sem(197),200:sem(141,159),
201:sem(*range(197,201)),202:sem(159,160,161,171,201),203:sem(*range(159,171)),
204:sem(171,172,148),205:sem(170),206:contract(*range(1,20))+sem(199,200),
207:contract(17)+sem(206),208:sem(200),209:sem(208),210:sem(200,201),
211:sem(201,206,208),212:sem(*range(202,208),*range(217,221)),213:sem(172,212),
214:["AF-ACE1-0001"]+sem(198,199,200),215:sem(214),
216:["AF-ACE1-%04d"%n for n in range(1,15)]+sem(214,215),
217:sem(181,214),218:sem(193,217),219:sem(*range(214,218)),
220:sem(194,211,*range(216,220)),221:sem(*range(214,220)),222:sem(211,214),
223:sem(186,187,221),224:sem(191,223),225:sem(176,193,224),226:sem(194,195,224),
227:sem(212,220,*range(223,227)),228:sem(189)+contract(17)}
def dependencies(rule_id):return sorted(set(DEPENDENCIES[int(rule_id.rsplit("-",1)[1])]))
